melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a2 b4 cis4 d2 \break

  % Line 2
  d2 cis2 b4 a4 b2 r8 a2 r2 \break

  % Line 3
  a2 a2 b4 cis4 d2 \break

  % Line 4
  a4 b4 a2 g2 fis2 r2 \break

  % Line 5
  a4 b4 a2 g2 fis2 \break

  % Line 6
  b4 a4 g4 fis4 e2 d1 \bar "|."
}
