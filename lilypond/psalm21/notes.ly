melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a4 g4 fis2 b2 a4 g4 fis2 r2 \break

  % Line 2
  d2 fis4 g4 a4 a4 b2 a2 r2 \break

  % Line 3
  a2 fis4 fis4 d4 fis4 e2 d2 r2 \break

  % Line 4
  a'2 b4 b4 cis2 a2 b4 cis4 d2 r2 \break

  % Line 5
  a2 b4 b4 a2 g2 fis2 r2 \break

  % Line 6
  d2 e4 g4 fis2 e2 d1 \bar "|."
}
