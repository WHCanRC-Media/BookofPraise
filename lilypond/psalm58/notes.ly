melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 b4 b4 a2 d2 cis4 d4 b2 a2 r2 \break

  % Line 2
  a2 fis4 fis4 d4 g4 fis2 e2 d2 r2 \break

  % Line 3
  d2 fis4 g4 a2 a2 b4 cis4 d2 r2 \break

  % Line 4
  d2 cis4 b4 a4 b4 a4 g4 fis2 e2 r2 \break

  % Line 5
  e2 fis4 g4 a2 fis2 b4 b4 a2 r2 \break

  % Line 6
  d2 cis4 b4 a4 d4 cis2 b2 a1 \bar "|."
}
