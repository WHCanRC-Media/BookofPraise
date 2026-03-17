melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'2 a4 g4 fis2 a2 b4 a4 g4 fis4 e2 r2 \break

  % Line 2
  e2 b'4 b4 a2 b2 cis4 d4 cis4 cis4 b2 r2 \break

  % Line 3
  b2 b4 b4 a2 fis2 g4 b4 a4 g4 fis2 r2 \break

  % Line 4
  b2 b4 b4 a2 fis2 g4 b4 a4 g4 fis4 \break

  % Line 5
  a4 g4 fis4 e2 d2 e4 fis4 g4 a4 b2 fis2 r2 \break

  % Line 6
  fis2 a4 a4 b2 d2 cis4 b4 a4 g4 fis2 e1 \bar "|."
}
